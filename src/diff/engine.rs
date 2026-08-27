use std::collections::HashMap;

use super::types::{ControlSnapshot, DiffEntry, DiffSummary, FieldChange};

/// Compare two control maps and produce a list of diff entries.
///
/// Walks both `old_map` and `new_map` to detect additions, removals, field
/// changes, and UUID-only changes. Results are sorted by control ID.
#[must_use]
pub fn compare_controls<SO, SN>(
    old_map: &HashMap<String, ControlSnapshot, SO>,
    new_map: &HashMap<String, ControlSnapshot, SN>,
) -> Vec<DiffEntry>
where
    SO: std::hash::BuildHasher,
    SN: std::hash::BuildHasher,
{
    let mut entries = Vec::new();

    for (id, new_snap) in new_map {
        if !old_map.contains_key(id) {
            entries
                .push(DiffEntry::Added { control_id: id.clone(), new_uuid: new_snap.uuid.clone() });
        }
    }

    for (id, old_snap) in old_map {
        let Some(new_snap) = new_map.get(id) else {
            entries.push(DiffEntry::Removed {
                control_id: id.clone(),
                old_uuid: old_snap.uuid.clone(),
            });
            continue;
        };

        let field_changes = compute_field_changes(old_snap, new_snap);
        let uuid_differs = old_snap.uuid != new_snap.uuid;

        match (uuid_differs, field_changes.is_empty()) {
            (true, true) => entries.push(DiffEntry::UuidChanged {
                control_id: id.clone(),
                old_uuid: old_snap.uuid.clone(),
                new_uuid: new_snap.uuid.clone(),
            }),
            (_, false) => entries.push(DiffEntry::Changed {
                control_id: id.clone(),
                old_uuid: old_snap.uuid.clone(),
                new_uuid: new_snap.uuid.clone(),
                field_changes,
            }),
            (false, true) => {}
        }
    }

    entries.sort_unstable_by(|a, b| a.control_id().cmp(b.control_id()));
    entries
}

fn compute_field_changes(old: &ControlSnapshot, new: &ControlSnapshot) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    if old.title != new.title {
        changes.push(FieldChange {
            field_name: "title".to_string(),
            old_value: old.title.clone(),
            new_value: new.title.clone(),
        });
    }

    if old.description != new.description {
        changes.push(FieldChange {
            field_name: "description".to_string(),
            old_value: old.description.clone(),
            new_value: new.description.clone(),
        });
    }

    // Preserve positional meaning while removing the unaffected common prefix
    // and suffix. Snapshot part IDs would provide a stronger long-term anchor.
    let common_prefix = old
        .parts_prose
        .iter()
        .zip(&new.parts_prose)
        .take_while(|(old_value, new_value)| old_value == new_value)
        .count();
    let common_suffix = old.parts_prose[common_prefix..]
        .iter()
        .rev()
        .zip(new.parts_prose[common_prefix..].iter().rev())
        .take_while(|(old_value, new_value)| old_value == new_value)
        .count();
    let old_changed = &old.parts_prose[common_prefix..old.parts_prose.len() - common_suffix];
    let new_changed = &new.parts_prose[common_prefix..new.parts_prose.len() - common_suffix];

    for offset in 0..old_changed.len().max(new_changed.len()) {
        let old_value = old_changed.get(offset);
        let new_value = new_changed.get(offset);
        if old_value != new_value {
            changes.push(FieldChange {
                field_name: format!("statement[{}]", common_prefix + offset),
                old_value: old_value.cloned(),
                new_value: new_value.cloned(),
            });
        }
    }

    changes
}

/// Build a [`DiffSummary`] from a list of diff entries.
///
/// Counts added, removed, changed, and UUID-change entries.
/// Computes `unchanged` as `total_old - (removed + changed + uuid_changes)`.
#[must_use]
pub fn build_summary(entries: &[DiffEntry], total_old: usize, total_new: usize) -> DiffSummary {
    let mut added = 0;
    let mut removed = 0;
    let mut changed = 0;
    let mut uuid_changes = 0;

    for entry in entries {
        match entry {
            DiffEntry::Added { .. } => added += 1,
            DiffEntry::Removed { .. } => removed += 1,
            DiffEntry::Changed { .. } => changed += 1,
            DiffEntry::UuidChanged { .. } => uuid_changes += 1,
        }
    }

    // unchanged = controls in old that were neither removed nor changed nor uuid-changed
    let unchanged = total_old.saturating_sub(removed + changed + uuid_changes);
    debug_assert_eq!(
        total_old,
        removed + changed + uuid_changes + unchanged,
        "old control total must match diff entry partition"
    );
    debug_assert_eq!(
        total_new,
        added + changed + uuid_changes + unchanged,
        "new control total must match diff entry partition"
    );

    DiffSummary { total_old, total_new, added, removed, changed, unchanged, uuid_changes }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(id: &str, uuid: &str, title: &str, prose: &[&str]) -> ControlSnapshot {
        ControlSnapshot {
            control_id: id.to_string(),
            uuid: Some(uuid.to_string()),
            title: Some(title.to_string()),
            description: None,
            parts_prose: prose.iter().map(std::string::ToString::to_string).collect(),
        }
    }

    fn to_map(snaps: Vec<ControlSnapshot>) -> HashMap<String, ControlSnapshot> {
        snaps.into_iter().map(|s| (s.control_id.clone(), s)).collect()
    }

    #[test]
    fn test_added_controls_detected() {
        let old = to_map(vec![snap("POL-AC-001", "", "Title1", &["Prose1"])]);
        let new = to_map(vec![
            snap("POL-AC-001", "", "Title1", &["Prose1"]),
            snap("POL-AC-002", "", "Title2", &["Prose2"]),
        ]);
        let entries = compare_controls(&old, &new);
        let added: Vec<_> =
            entries.iter().filter(|e| matches!(e, DiffEntry::Added { .. })).collect();
        assert_eq!(added.len(), 1);
        if let DiffEntry::Added { control_id, .. } = &added[0] {
            assert_eq!(control_id, "POL-AC-002");
        }
    }

    #[test]
    fn test_removed_controls_detected() {
        let old = to_map(vec![
            snap("POL-AC-001", "", "Title1", &["Prose1"]),
            snap("POL-AC-002", "", "Title2", &["Prose2"]),
        ]);
        let new = to_map(vec![snap("POL-AC-001", "", "Title1", &["Prose1"])]);
        let entries = compare_controls(&old, &new);
        let removed: Vec<_> =
            entries.iter().filter(|e| matches!(e, DiffEntry::Removed { .. })).collect();
        assert_eq!(removed.len(), 1);
        if let DiffEntry::Removed { control_id, .. } = &removed[0] {
            assert_eq!(control_id, "POL-AC-002");
        }
    }

    #[test]
    fn test_changed_controls_with_field_detail() {
        let old = to_map(vec![snap("POL-AC-001", "", "Old Title", &["Old Prose"])]);
        let new = to_map(vec![snap("POL-AC-001", "", "New Title", &["New Prose"])]);
        let entries = compare_controls(&old, &new);
        assert_eq!(entries.len(), 1);
        if let DiffEntry::Changed { control_id, field_changes, .. } = &entries[0] {
            assert_eq!(control_id, "POL-AC-001");
            assert!(!entries[0].uuid_changed());
            let field_names: Vec<_> =
                field_changes.iter().map(|fc| fc.field_name.as_str()).collect();
            assert!(field_names.contains(&"title"));
            assert!(field_names.contains(&"statement[0]"));
        } else {
            panic!("Expected Changed entry");
        }
    }

    #[test]
    fn test_identical_files_no_differences() {
        let old = to_map(vec![snap("POL-AC-001", "", "Title", &["Prose"])]);
        let new = to_map(vec![snap("POL-AC-001", "", "Title", &["Prose"])]);
        let entries = compare_controls(&old, &new);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_empty_old_all_added() {
        let old = HashMap::new();
        let new =
            to_map(vec![snap("POL-AC-001", "", "T1", &[]), snap("POL-AC-002", "", "T2", &[])]);
        let entries = compare_controls(&old, &new);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| matches!(e, DiffEntry::Added { .. })));
    }

    #[test]
    fn test_empty_new_all_removed() {
        let old =
            to_map(vec![snap("POL-AC-001", "", "T1", &[]), snap("POL-AC-002", "", "T2", &[])]);
        let new = HashMap::new();
        let entries = compare_controls(&old, &new);
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| matches!(e, DiffEntry::Removed { .. })));
    }

    #[test]
    fn test_title_only_change_reported() {
        let old = to_map(vec![snap("POL-AC-001", "", "Old Title", &["Same Prose"])]);
        let new = to_map(vec![snap("POL-AC-001", "", "New Title", &["Same Prose"])]);
        let entries = compare_controls(&old, &new);
        assert_eq!(entries.len(), 1);
        if let DiffEntry::Changed { field_changes, .. } = &entries[0] {
            assert_eq!(field_changes.len(), 1);
            assert_eq!(field_changes[0].field_name, "title");
        } else {
            panic!("Expected Changed entry");
        }
    }

    #[test]
    fn statement_prepend_reports_only_inserted_statement() {
        let old = to_map(vec![snap("AC-1", "", "Title", &["second", "third"])]);
        let new = to_map(vec![snap("AC-1", "", "Title", &["first", "second", "third"])]);

        let entries = compare_controls(&old, &new);
        let DiffEntry::Changed { field_changes, .. } = &entries[0] else {
            panic!("Expected Changed entry");
        };
        assert_eq!(field_changes.len(), 1);
        assert_eq!(field_changes[0].field_name, "statement[0]");
        assert_eq!(field_changes[0].old_value, None);
        assert_eq!(field_changes[0].new_value.as_deref(), Some("first"));
    }

    #[test]
    fn statement_middle_insertion_has_bounded_changes() {
        let old = to_map(vec![snap("AC-1", "", "Title", &["first", "third"])]);
        let new = to_map(vec![snap("AC-1", "", "Title", &["first", "second", "third"])]);

        let entries = compare_controls(&old, &new);
        let DiffEntry::Changed { field_changes, .. } = &entries[0] else {
            panic!("Expected Changed entry");
        };
        assert_eq!(field_changes.len(), 1);
        assert_eq!(field_changes[0].field_name, "statement[1]");
    }

    #[test]
    fn test_same_uuid_different_content_is_changed() {
        let old = to_map(vec![snap("POL-AC-001", "same-uuid", "Old Title", &[])]);
        let new = to_map(vec![snap("POL-AC-001", "same-uuid", "New Title", &[])]);
        let entries = compare_controls(&old, &new);
        assert_eq!(entries.len(), 1);
        if matches!(&entries[0], DiffEntry::Changed { .. }) {
            assert!(!entries[0].uuid_changed());
        } else {
            panic!("Expected Changed entry");
        }
    }

    #[test]
    fn test_entries_sorted_by_control_id() {
        let old = HashMap::new();
        let new = to_map(vec![
            snap("ZZZ-001", "", "Z", &[]),
            snap("AAA-001", "", "A", &[]),
            snap("MMM-001", "", "M", &[]),
        ]);
        let entries = compare_controls(&old, &new);
        let ids: Vec<_> = entries.iter().map(DiffEntry::control_id).collect();
        assert_eq!(ids, vec!["AAA-001", "MMM-001", "ZZZ-001"]);
    }

    #[test]
    fn test_bulk_change_scenario() {
        let mut old_snaps = Vec::new();
        let mut new_snaps = Vec::new();
        for i in 0..12 {
            let id = format!("POL-{i:03}");
            old_snaps.push(snap(&id, "", &format!("Old {i}"), &[&format!("OldProse {i}")]));
            new_snaps.push(snap(&id, "", &format!("New {i}"), &[&format!("NewProse {i}")]));
        }
        let entries = compare_controls(&to_map(old_snaps), &to_map(new_snaps));
        assert_eq!(entries.len(), 12);
        assert!(entries.iter().all(|e| matches!(e, DiffEntry::Changed { .. })));
        let ids: Vec<_> = entries.iter().map(DiffEntry::control_id).collect();
        let mut sorted_ids = ids.clone();
        sorted_ids.sort_unstable();
        assert_eq!(ids, sorted_ids);
    }

    #[test]
    fn test_build_summary_counts() {
        let entries = vec![
            DiffEntry::Added { control_id: "A".into(), new_uuid: None },
            DiffEntry::Added { control_id: "B".into(), new_uuid: None },
            DiffEntry::Removed { control_id: "C".into(), old_uuid: None },
            DiffEntry::Changed {
                control_id: "D".into(),
                old_uuid: None,
                new_uuid: None,
                field_changes: vec![FieldChange {
                    field_name: "title".into(),
                    old_value: Some("old".into()),
                    new_value: Some("new".into()),
                }],
            },
        ];
        let summary = build_summary(&entries, 5, 6);
        assert_eq!(summary.added, 2);
        assert_eq!(summary.removed, 1);
        assert_eq!(summary.changed, 1);
        assert_eq!(summary.total_old, 5);
        assert_eq!(summary.total_new, 6);
        assert_eq!(summary.unchanged, 3);
        assert_eq!(summary.uuid_changes, 0);
    }

    // --- US2: UUID stability tests (T019) ---

    // AC-5: UuidChanged emitted when UUID differs but fields are same
    #[test]
    fn test_uuid_stability_change_detected() {
        let old = to_map(vec![snap("POL-AC-001", "old-uuid", "Title", &["Prose"])]);
        let new = to_map(vec![snap("POL-AC-001", "new-uuid", "Title", &["Prose"])]);
        let entries = compare_controls(&old, &new);
        assert_eq!(entries.len(), 1);
        assert!(matches!(
            &entries[0],
            DiffEntry::UuidChanged { control_id, old_uuid, new_uuid }
            if control_id == "POL-AC-001"
                && old_uuid.as_deref() == Some("old-uuid")
                && new_uuid.as_deref() == Some("new-uuid")
        ));
    }

    // No UUID changes when UUIDs are identical
    #[test]
    fn test_no_uuid_change_when_identical() {
        let old = to_map(vec![snap("POL-AC-001", "same-uuid", "Title", &["Prose"])]);
        let new = to_map(vec![snap("POL-AC-001", "same-uuid", "Title", &["Prose"])]);
        let entries = compare_controls(&old, &new);
        assert!(entries.is_empty());
    }

    // Co-occurrence: UUID and fields changed → Changed with a derived UUID change.
    // It does not produce a UuidChanged entry or increment uuid_changes.
    #[test]
    fn test_co_occurrence_uuid_and_field_changes() {
        let old = to_map(vec![snap("POL-AC-001", "old-uuid", "Old Title", &["Old Prose"])]);
        let new = to_map(vec![snap("POL-AC-001", "new-uuid", "New Title", &["New Prose"])]);
        let entries = compare_controls(&old, &new);
        assert_eq!(entries.len(), 1);
        if let DiffEntry::Changed { field_changes, .. } = &entries[0] {
            assert!(entries[0].uuid_changed());
            assert!(!field_changes.is_empty());
        } else {
            panic!("Expected Changed entry, not UuidChanged");
        }
        // build_summary should NOT count this as uuid_changes
        let summary = build_summary(&entries, 1, 1);
        assert_eq!(summary.uuid_changes, 0);
        assert_eq!(summary.changed, 1);
    }

    // build_summary correctly counts standalone UuidChanged entries
    #[test]
    fn test_build_summary_uuid_changes_count() {
        let entries = vec![
            DiffEntry::UuidChanged {
                control_id: "A".into(),
                old_uuid: Some("old".into()),
                new_uuid: Some("new".into()),
            },
            DiffEntry::Changed {
                control_id: "B".into(),
                old_uuid: Some("old2".into()),
                new_uuid: Some("new2".into()),
                field_changes: vec![FieldChange {
                    field_name: "title".into(),
                    old_value: Some("x".into()),
                    new_value: Some("y".into()),
                }],
            },
        ];
        let summary = build_summary(&entries, 3, 3);
        assert_eq!(summary.uuid_changes, 1); // Only UuidChanged entries count here.
        assert_eq!(summary.changed, 1);
    }

    #[test]
    fn test_description_field_change_label() {
        let old = to_map(vec![ControlSnapshot {
            control_id: "IR-001".into(),
            uuid: None,
            title: None,
            description: Some("Old desc".into()),
            parts_prose: vec![],
        }]);
        let new = to_map(vec![ControlSnapshot {
            control_id: "IR-001".into(),
            uuid: None,
            title: None,
            description: Some("New desc".into()),
            parts_prose: vec![],
        }]);
        let entries = compare_controls(&old, &new);
        assert_eq!(entries.len(), 1);
        if let DiffEntry::Changed { field_changes, .. } = &entries[0] {
            assert_eq!(field_changes.len(), 1);
            assert_eq!(field_changes[0].field_name, "description");
        } else {
            panic!("Expected Changed entry");
        }
    }

    #[test]
    fn aggregated_component_descriptions_remain_visible_in_diff() {
        let old = to_map(vec![ControlSnapshot {
            control_id: "AC-1".into(),
            uuid: None,
            title: None,
            description: Some("First implementation\nSecond implementation".into()),
            parts_prose: vec![],
        }]);
        let new = to_map(vec![ControlSnapshot {
            control_id: "AC-1".into(),
            uuid: None,
            title: None,
            description: Some("First implementation\nUpdated implementation".into()),
            parts_prose: vec![],
        }]);

        let entries = compare_controls(&old, &new);
        let DiffEntry::Changed { field_changes, .. } = &entries[0] else {
            panic!("Expected Changed entry");
        };
        assert_eq!(
            field_changes[0].old_value.as_deref(),
            Some("First implementation\nSecond implementation")
        );
        assert_eq!(
            field_changes[0].new_value.as_deref(),
            Some("First implementation\nUpdated implementation")
        );
    }
}

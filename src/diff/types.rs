use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactType {
    Catalog,
    ComponentDefinition,
}

impl fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog => write!(f, "Catalog"),
            Self::ComponentDefinition => write!(f, "ComponentDefinition"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlSnapshot {
    pub control_id: String,
    pub uuid: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub parts_prose: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChange {
    pub field_name: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffEntry {
    Added {
        control_id: String,
        new_uuid: String,
    },
    Removed {
        control_id: String,
        old_uuid: String,
    },
    Changed {
        control_id: String,
        old_uuid: String,
        new_uuid: String,
        uuid_changed: bool,
        field_changes: Vec<FieldChange>,
    },
    UuidChanged {
        control_id: String,
        old_uuid: String,
        new_uuid: String,
    },
}

impl DiffEntry {
    #[must_use]
    pub fn control_id(&self) -> &str {
        match self {
            Self::Added { control_id, .. }
            | Self::Removed { control_id, .. }
            | Self::Changed { control_id, .. }
            | Self::UuidChanged { control_id, .. } => control_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffSummary {
    pub total_old: usize,
    pub total_new: usize,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
    pub unchanged: usize,
    pub uuid_changes: usize,
}

impl DiffSummary {
    #[must_use]
    pub fn has_changes(&self) -> bool {
        self.added > 0 || self.removed > 0 || self.changed > 0 || self.uuid_changes > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffReport {
    pub old_file: String,
    pub new_file: String,
    pub artifact_type: ArtifactType,
    pub entries: Vec<DiffEntry>,
    pub summary: DiffSummary,
}

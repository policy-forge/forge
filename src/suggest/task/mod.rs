//! Versioned task schemas for the two v1 suggestion tasks.
//!
//! Both tasks share one pipeline. The task kind and its schema version are
//! recorded on every request and response, and the runtime rejects a version
//! that does not match its kind, so a document can never be decoded as the
//! wrong task.

pub mod drafting;
pub mod mapping;

use serde::{Deserialize, Serialize};

use super::shared;

/// Maximum suggestions, citations, assumptions or unresolved questions in one
/// response or bundle.
pub const MAX_SUGGESTIONS: usize = 1_000;

/// Which versioned task a request or response carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskKind {
    /// Mapping candidates for an existing review workflow.
    MappingCandidates,
    /// Clause drafting against approved authoring context.
    PolicyDrafting,
}

impl TaskKind {
    /// Stable wire spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MappingCandidates => "mapping-candidates",
            Self::PolicyDrafting => "policy-drafting",
        }
    }

    /// The one schema version this kind is decoded with.
    #[must_use]
    pub const fn schema_version(self) -> &'static str {
        match self {
            Self::MappingCandidates => mapping::TASK_SCHEMA_VERSION,
            Self::PolicyDrafting => drafting::TASK_SCHEMA_VERSION,
        }
    }
}

/// Reject a task schema version that does not belong to its declared kind.
pub(in crate::suggest) fn check_task_version(
    name: &str,
    kind: TaskKind,
    version: &str,
) -> Result<(), crate::ForgeError> {
    if version != kind.schema_version() {
        return Err(shared::error(format!(
            "{name}.schema_version must be {} for task kind {}",
            kind.schema_version(),
            kind.as_str()
        )));
    }
    Ok(())
}

/// The task one document was produced for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskIdentity {
    /// Task kind.
    pub kind: TaskKind,
    /// Task schema version; must match the kind.
    pub schema_version: String,
}

impl TaskIdentity {
    /// Validate that the version belongs to the declared kind.
    pub(in crate::suggest) fn validate(&self, name: &str) -> Result<(), crate::ForgeError> {
        check_task_version(name, self.kind, &self.schema_version)
    }
}

/// One citation naming an allowlisted context unit.
///
/// The unit identifier is the only citation target; a quote, when present, must
/// be byte-identical to the cited unit's text, which is what makes an altered
/// citation detectable rather than plausible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    /// Identifier of an allowlisted unit in the same request.
    pub unit_id: String,
    /// Optional verbatim copy of the cited unit's text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<String>,
}

impl Citation {
    fn validate(&self, name: &str) -> Result<(), crate::ForgeError> {
        shared::key(&format!("{name}.unit_id"), &self.unit_id)?;
        if let Some(quote) = &self.quote {
            shared::text(&format!("{name}.quote"), quote)?;
        }
        Ok(())
    }
}

/// Validate a citation list.
pub(in crate::suggest) fn citations(
    name: &str,
    values: &[Citation],
) -> Result<(), crate::ForgeError> {
    if values.len() > MAX_SUGGESTIONS {
        return Err(shared::error(format!("{name} exceeds {MAX_SUGGESTIONS} entries")));
    }
    let mut seen = std::collections::BTreeSet::new();
    for (index, citation) in values.iter().enumerate() {
        citation.validate(&format!("{name}[{index}]"))?;
        if !seen.insert(citation.unit_id.as_str()) {
            return Err(shared::error(format!("{name} cites the same unit more than once")));
        }
    }
    Ok(())
}

/// One validated task payload inside a quarantine bundle or disposition.
///
/// Exactly one arm is present; which one is decided by the bundle's task kind,
/// so a record can never carry a payload from the other task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuggestionBody {
    /// Mapping-candidate payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mapping: Option<mapping::MappingCandidate>,
    /// Policy-drafting payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drafting: Option<drafting::DraftClause>,
}

impl SuggestionBody {
    /// Validate that this body carries exactly the payload of `kind`.
    ///
    /// # Errors
    /// Returns an authoring error when the body is empty, carries both arms, or
    /// carries the arm of the other task.
    pub(in crate::suggest) fn validate(
        &self,
        name: &str,
        kind: TaskKind,
    ) -> Result<(), crate::ForgeError> {
        match (kind, &self.mapping, &self.drafting) {
            (TaskKind::MappingCandidates, Some(candidate), None) => {
                candidate.validate(&format!("{name}.mapping"))
            }
            (TaskKind::PolicyDrafting, None, Some(clause)) => {
                clause.validate(&format!("{name}.drafting"))
            }
            (TaskKind::MappingCandidates, None, _) => Err(shared::error(format!(
                "{name} must carry the mapping candidate for a mapping-candidate task"
            ))),
            (TaskKind::PolicyDrafting, _, None) => Err(shared::error(format!(
                "{name} must carry the draft clause for a policy-drafting task"
            ))),
            _ => Err(shared::error(format!("{name} must carry exactly one task payload"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ForgeError;

    fn citation(unit_id: &str) -> Citation {
        Citation { unit_id: unit_id.to_string(), quote: None }
    }

    #[test]
    fn task_kinds_map_to_their_own_schema_versions() {
        assert_eq!(TaskKind::MappingCandidates.schema_version(), "forge.suggest-task-mapping/1");
        assert_eq!(TaskKind::PolicyDrafting.schema_version(), "forge.suggest-task-drafting/1");
        assert!(
            check_task_version(
                "request.task",
                TaskKind::MappingCandidates,
                mapping::TASK_SCHEMA_VERSION
            )
            .is_ok()
        );
        assert!(
            check_task_version(
                "request.task",
                TaskKind::MappingCandidates,
                drafting::TASK_SCHEMA_VERSION
            )
            .is_err()
        );
    }

    #[test]
    fn citations_reject_duplicate_units_and_unsafe_identifiers() {
        assert!(citations("citations", &[citation("unit-1"), citation("unit-2")]).is_ok());
        assert!(citations("citations", &[citation("unit-1"), citation("unit-1")]).is_err());
        assert!(citations("citations", &[citation("Unit-1")]).is_err());
        let mut quoted = citation("unit-1");
        quoted.quote = Some("copy of the unit".to_string());
        assert!(citations("citations", &[quoted]).is_ok());
        let mut blank = citation("unit-1");
        blank.quote = Some("   ".to_string());
        assert!(citations("citations", &[blank]).is_err());
        assert!(matches!(
            citations("citations", &[citation("Unit-1")]),
            Err(ForgeError::Authoring(_))
        ));
    }
}

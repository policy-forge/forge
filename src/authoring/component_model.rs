//! Closed opt-in component bindings and value-redacted captured evidence.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::manifest::{AnswerPin, PinnedFile, Review, Sensitivity};
use super::model::{AnswerStatus, AuthoringPlan, DraftState};
use super::render::ByteSpan;
use crate::policy::manifest::{ComponentStatus, ParameterValue};

pub const COMPONENTS_SCHEMA_VERSION: &str = "forge.author-components/1";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorComponents {
    pub schema_version: String,
    pub project_sha256: String,
    pub instances: Vec<AuthorComponent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorComponent {
    pub instance_key: String,
    pub policy_key: String,
    pub topic_key: String,
    pub gap_ids: Vec<String>,
    pub component_manifest: PinnedFile,
    pub source: PinnedFile,
    pub parameters: BTreeMap<String, ParameterBinding>,
    pub review: Review,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ParameterBinding {
    Literal { value: ParameterValue, sensitivity: Sensitivity },
    Answer { question_key: String, answer_key: String, expected_sha256: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentEvidence {
    pub instance_key: String,
    pub policy_key: String,
    pub topic_key: String,
    pub gap_ids: Vec<String>,
    pub component_manifest: PinnedFile,
    pub source: PinnedFile,
    pub component_key: String,
    pub version: String,
    pub status: ComponentStatus,
    pub review: Review,
    pub bindings: Vec<BindingEvidence>,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BindingEvidence {
    pub parameter_name: String,
    pub value_sha256: Option<String>,
    pub answer_ref: Option<AnswerPin>,
    pub observed_answer_sha256: Option<String>,
    pub question_key: Option<String>,
    pub sensitivity: Sensitivity,
    pub answer_state: Option<AnswerStatus>,
}

/// Separate Phase 2 origin contract: no extension of a Phase 1 exhaustive enum.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentOrigin {
    ComponentSource,
    Parameter,
    GeneratedNewline,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentSpan {
    pub output: ByteSpan,
    pub kind: ComponentOrigin,
    pub source: Option<ByteSpan>,
    pub parameter_name: Option<String>,
    pub parameter_value_sha256: Option<String>,
    pub answer_ref: Option<AnswerPin>,
}

#[derive(Debug, Clone)]
pub struct ComponentFragment {
    pub markdown: Vec<u8>,
    pub spans: Vec<ComponentSpan>,
}

#[derive(Debug, Clone)]
pub struct LoadedInstance {
    pub evidence: ComponentEvidence,
    pub fragment: Option<ComponentFragment>,
}

#[derive(Debug, Clone)]
pub struct LoadedComponents {
    pub manifest_sha256: String,
    pub instances: BTreeMap<String, LoadedInstance>,
}

impl LoadedComponents {
    /// A missing bound value blocks all content in its explicitly assigned section.
    /// Component content never sets the human-draft-present state.
    pub fn apply_plan(&self, plan: &mut AuthoringPlan) {
        let blocked: BTreeSet<_> = self
            .instances
            .values()
            .filter(|instance| instance.evidence.blocked)
            .map(|instance| {
                (instance.evidence.policy_key.as_str(), instance.evidence.topic_key.as_str())
            })
            .collect();
        for policy in &mut plan.policies {
            for section in &mut policy.sections {
                if blocked.contains(&(policy.policy_key.as_str(), section.topic_key.as_str())) {
                    section.state = DraftState::BlockedContext;
                }
            }
            if policy.sections.iter().any(|section| section.state == DraftState::BlockedContext) {
                policy.state = DraftState::BlockedContext;
            }
        }
    }
}

//! Deterministic local drafting plans and policy skeletons from explicit human inputs.

pub mod component_model;
pub mod components;
mod generation;
pub mod handoff;
pub mod html;
pub mod impact;
pub(crate) mod input;

pub use generation::{execute_extended, execute_handoff, execute_impact};
mod scaffold;
pub use scaffold::execute as execute_scaffold;
pub mod manifest;
pub mod model;
pub mod output;
pub mod plan;
pub mod render;
pub mod report;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::ForgeError;
use crate::cli::AuthorReportFormat;
use model::{AuthoringPlan, DraftState};

pub(crate) fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Authoring(message.into())
}

/// The drafting plan, project root and question prompts a reuse run needs.
///
/// The plan already carries every unresolved section; prompts are the one
/// pack value it omits, so they are passed alongside instead of widening the
/// frozen `/1` plan contract.
pub(crate) struct ReuseSeed {
    pub(crate) root: PathBuf,
    pub(crate) plan: AuthoringPlan,
    pub(crate) prompts: BTreeMap<String, String>,
    /// Captured project inputs, retained so reuse can revalidate them before
    /// publishing a generation.
    pub(crate) captures: input::CaptureSet,
}

/// Validate a project and return the plan, root, prompts and captured inputs.
///
/// # Errors
/// Returns an authoring error for invalid inputs, stale pins, or unsafe paths.
pub(crate) fn reuse_seed(manifest: &Path) -> Result<ReuseSeed, ForgeError> {
    let prepared = input::prepare(manifest)?;
    let plan = plan::build_plan(&prepared.loaded)?;
    let prompts = prepared
        .loaded
        .pack
        .questions
        .iter()
        .map(|question| (question.key.clone(), question.prompt.clone()))
        .collect();
    Ok(ReuseSeed { root: prepared.root, plan, prompts, captures: prepared.captures })
}

/// Validate a complete project and return its deterministic drafting plan without writing.
///
/// # Errors
/// Returns an authoring error for invalid inputs, stale pins, or unsafe paths.
pub fn prepare_plan(manifest: &Path) -> Result<AuthoringPlan, ForgeError> {
    let prepared = input::prepare(manifest)?;
    plan::build_plan(&prepared.loaded)
}

/// Plan or build one complete output generation from a confined input snapshot.
///
/// # Errors
/// Returns an authoring error before publication for invalid input or unsafe output.
/// A durability error after the atomic directory rename can leave a complete generation.
pub fn execute(
    manifest: &Path,
    build: bool,
    output_dir: Option<&Path>,
    format: &AuthorReportFormat,
) -> Result<bool, ForgeError> {
    if build && output_dir.is_none() {
        return Err(error("author build requires --output-dir"));
    }
    let prepared = input::prepare(manifest)?;
    let plan = plan::build_plan(&prepared.loaded)?;
    let json = report::render_json(&plan)?;
    let text = report::render_text(&plan);
    let stdout = match format {
        AuthorReportFormat::Text => text.clone(),
        AuthorReportFormat::Json => String::from_utf8(json.clone())
            .map_err(|cause| error(format!("plan JSON encoding: {cause}")))?,
    };
    let mut artifacts = vec![
        output::OutputArtifact { relative_path: "plan.json".to_owned(), bytes: json },
        output::OutputArtifact { relative_path: "plan.txt".to_owned(), bytes: text.into_bytes() },
    ];
    if build {
        let rendered = render::render(&prepared.loaded, &plan)?;
        artifacts.push(output::OutputArtifact {
            relative_path: "provenance.json".to_owned(),
            bytes: rendered.provenance,
        });
        for policy in rendered.policies {
            artifacts.push(output::OutputArtifact {
                relative_path: policy.relative_path,
                bytes: policy.markdown,
            });
        }
    }
    prepared.verify_inputs()?;
    if let Some(destination) = output_dir {
        output::publish(&prepared.root, destination, &artifacts)?;
    }
    crate::cli::output::write_output(&stdout, None)
        .map_err(|cause| error(format!("cannot write authoring report: {cause}")))?;
    Ok(action_required(&plan))
}

fn action_required(plan: &AuthoringPlan) -> bool {
    plan.counts.unresolved > 0
        || plan.policies.iter().any(|policy| policy.state == DraftState::BlockedContext)
}

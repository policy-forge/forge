//! Deterministic local drafting plans and policy skeletons from explicit human inputs.

mod input;
pub mod manifest;
pub mod model;
pub mod output;
pub mod plan;
pub mod render;
pub mod report;

use std::path::Path;

use crate::ForgeError;
use crate::cli::AuthorReportFormat;
use model::{AuthoringPlan, DraftState};

pub(crate) fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Authoring(message.into())
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

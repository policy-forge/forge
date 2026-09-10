//! Coordinator-owned capture, replay and single-generation publication.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::{
    component_model, components, error, handoff, html, impact, input, manifest, model, output,
    plan, render, report,
};
use crate::ForgeError;
use crate::cli::AuthorReportFormat;
use crate::hashing::sha256_hex;

struct Generation {
    prepared: input::PreparedProject,
    plan: model::AuthoringPlan,
    components: Option<component_model::LoadedComponents>,
    rendered: Option<render::RenderedAuthorProject>,
    artifacts: Vec<output::OutputArtifact>,
    json: Vec<u8>,
    text: String,
}

#[derive(Serialize)]
struct ComponentPlan<'a> {
    schema_version: &'static str,
    plan: &'a model::AuthoringPlan,
    component_manifest_sha256: &'a str,
    components: Vec<&'a component_model::ComponentEvidence>,
}

// Keep capture, bounded rendering and artifact order visible in one orchestration pass.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn prepare_generation(
    manifest_path: &Path,
    component_path: Option<&Path>,
    build: bool,
    include_html: bool,
    budget: u64,
    phase: &Cell<impact::UnverifiedReason>,
    expected: Option<(&str, Option<&str>)>,
) -> Result<Generation, ForgeError> {
    phase.set(impact::UnverifiedReason::SnapshotInput);
    let mut prepared = input::prepare_pinned(manifest_path, budget, expected.map(|pins| pins.0))?;
    let mut plan = plan::build_plan(&prepared.loaded)?;
    let components = if let Some(path) = component_path {
        phase.set(impact::UnverifiedReason::ComponentInput);
        manifest::validate_local_path("components", path)?;
        let bytes = prepared.captures.read(
            "author-components",
            path,
            expected.and_then(|pins| pins.1),
            manifest::MAX_MANIFEST_BYTES,
        )?;
        let loaded = &prepared.loaded;
        let components = components::prepare(&bytes, loaded, &plan, |role, pin, limit| {
            prepared.captures.pinned(role, pin, limit)
        })?;
        components.apply_plan(&mut plan);
        prepared.loaded.inputs = prepared.captures.fingerprints();
        plan.provenance.inputs.clone_from(&prepared.loaded.inputs);
        Some(components)
    } else {
        None
    };
    phase.set(impact::UnverifiedReason::RenderedOutput);
    let json = if let Some(items) = &components {
        report::encode(&ComponentPlan {
            schema_version: "forge.authoring-plan/2",
            plan: &plan,
            component_manifest_sha256: &items.manifest_sha256,
            components: items.instances.values().map(|instance| &instance.evidence).collect(),
        })?
    } else {
        report::render_json(&plan)?
    };
    let mut text =
        report::render_text_bounded(&plan, output::MAX_OUTPUT_BYTES.saturating_sub(json.len()))?;
    if let Some(items) = &components {
        append_text(
            &mut text,
            "Component extension: explicitly supplied drafting content; status is asserted provenance.\n",
            output::MAX_OUTPUT_BYTES.saturating_sub(json.len()),
        )?;
        for item in items.instances.values() {
            let e = &item.evidence;
            append_text(
                &mut text,
                &format!(
                    "- instance={} policy={} topic={} blocked={} source-sha256={}\n",
                    e.instance_key, e.policy_key, e.topic_key, e.blocked, e.source.expected_sha256
                ),
                output::MAX_OUTPUT_BYTES.saturating_sub(json.len()),
            )?;
        }
    }
    let mut artifacts =
        vec![artifact("plan.json", json.clone()), artifact("plan.txt", text.as_bytes().to_vec())];
    let rendered = if build {
        let rendered = if let Some(items) = &components {
            render::render_with_components(
                &prepared.loaded,
                &plan,
                items,
                &json,
                remaining(&artifacts)?,
            )?
        } else {
            render::render_bounded(&prepared.loaded, &plan, remaining(&artifacts)?)?
        };
        artifacts.push(artifact("provenance.json", rendered.provenance.clone()));
        for policy in &rendered.policies {
            artifacts.push(artifact(&policy.relative_path, policy.markdown.clone()));
        }
        if let Some(items) = &components {
            #[derive(Serialize)]
            struct Lock<'a> {
                schema_version: &'static str,
                project_sha256: &'a str,
                component_manifest_sha256: &'a str,
                plan_sha256: String,
                provenance_sha256: String,
                policies: BTreeMap<&'a str, String>,
                components: Vec<&'a component_model::ComponentEvidence>,
            }
            artifacts.push(artifact(
                "components.lock.json",
                report::encode_bounded(
                    &Lock {
                        schema_version: "forge.authoring-component-lock/1",
                        project_sha256: &prepared.loaded.project_sha256,
                        component_manifest_sha256: &items.manifest_sha256,
                        plan_sha256: sha256_hex(&json),
                        provenance_sha256: sha256_hex(&rendered.provenance),
                        policies: rendered
                            .policies
                            .iter()
                            .map(|policy| {
                                (policy.policy_key.as_str(), sha256_hex(&policy.markdown))
                            })
                            .collect(),
                        components: items.instances.values().map(|item| &item.evidence).collect(),
                    },
                    remaining(&artifacts)?,
                )?,
            ));
        }
        if include_html {
            artifacts.push(artifact(
                "provenance.html",
                html::render_provenance_bounded(&rendered.provenance, remaining(&artifacts)?)?,
            ));
        }
        Some(rendered)
    } else {
        None
    };
    if include_html {
        let bytes = if components.is_some() {
            html::render_component_plan_bounded(&json, remaining(&artifacts)?)?
        } else {
            html::render_plan_bounded(&plan, remaining(&artifacts)?)?
        };
        artifacts.push(artifact("plan.html", bytes));
    }
    check_artifacts(&artifacts)?;
    Ok(Generation { prepared, plan, components, rendered, artifacts, json, text })
}

fn append_text(output: &mut String, text: &str, limit: usize) -> Result<(), ForgeError> {
    if text.len() > limit.saturating_sub(output.len()) {
        return Err(error("text report exceeds remaining generation budget"));
    }
    output.push_str(text);
    Ok(())
}

fn remaining(artifacts: &[output::OutputArtifact]) -> Result<usize, ForgeError> {
    check_artifacts(artifacts)?;
    Ok(output::MAX_OUTPUT_BYTES - artifacts.iter().map(|item| item.bytes.len()).sum::<usize>())
}

fn artifact(path: &str, bytes: Vec<u8>) -> output::OutputArtifact {
    output::OutputArtifact { relative_path: path.to_owned(), bytes }
}

fn check_artifacts(artifacts: &[output::OutputArtifact]) -> Result<(), ForgeError> {
    if artifacts.len() > 2048 {
        return Err(error("authoring generation exceeds artifact count"));
    }
    let mut total = 0usize;
    for item in artifacts {
        total = total.checked_add(item.bytes.len()).ok_or_else(|| error("output size overflow"))?;
        if total > output::MAX_OUTPUT_BYTES {
            return Err(error("complete authoring generation exceeds 50 MiB"));
        }
    }
    Ok(())
}

fn print_report(json: &[u8], text: &str, format: AuthorReportFormat) -> Result<(), ForgeError> {
    let stdout = match format {
        AuthorReportFormat::Text => text,
        AuthorReportFormat::Json => {
            std::str::from_utf8(json).map_err(|_| error("invalid report UTF-8"))?
        }
    };
    crate::cli::output::write_output(stdout, None)
        .map_err(|cause| error(format!("cannot write authoring report: {cause}")))
}

/// Opt-in extension of planning/building; omitted controls retain Phase 1 behavior.
/// # Errors
/// Rejects invalid input, drift, unsafe paths, or publication failure.
pub fn execute_extended(
    manifest: &Path,
    build: bool,
    destination: Option<&Path>,
    format: &AuthorReportFormat,
    components: Option<&Path>,
    include_html: bool,
) -> Result<bool, ForgeError> {
    if components.is_none() && !include_html {
        return super::execute(manifest, build, destination, format);
    }
    if (build || include_html) && destination.is_none() {
        return Err(error("build and HTML require --output-dir"));
    }
    let generation = prepare_generation(
        manifest,
        components,
        build,
        include_html,
        manifest::MAX_TOTAL_BYTES,
        &Cell::new(impact::UnverifiedReason::SnapshotInput),
        None,
    )?;
    generation.prepared.verify_inputs()?;
    if let Some(destination) = destination {
        output::publish(&generation.prepared.root, destination, &generation.artifacts)?;
    }
    print_report(&generation.json, &generation.text, *format)?;
    Ok(super::action_required(&generation.plan))
}

fn capture_request(path: &Path) -> Result<(PathBuf, input::CaptureSet, Vec<u8>), ForgeError> {
    let absolute = input::absolute_manifest_path(path)?;
    let root = absolute
        .parent()
        .ok_or_else(|| error("request requires a parent directory"))?
        .to_path_buf();
    let name = absolute.file_name().ok_or_else(|| error("request must name a file"))?;
    let mut captures = input::CaptureSet::new(root.clone());
    let bytes =
        captures.read("author-request", Path::new(name), None, manifest::MAX_MANIFEST_BYTES)?;
    Ok((root, captures, bytes))
}

fn relative_component(
    project: &manifest::PinnedFile,
    component: &manifest::PinnedFile,
) -> Result<PathBuf, ForgeError> {
    let parent = project.path.parent().unwrap_or_else(|| Path::new(""));
    let relative = component
        .path
        .strip_prefix(parent)
        .map_err(|_| error("components must be within the author project directory"))?;
    manifest::validate_local_path("components", relative)?;
    Ok(relative.to_path_buf())
}

#[allow(clippy::too_many_arguments)]
fn pinned_generation(
    root: &Path,
    captures: &mut input::CaptureSet,
    project: &manifest::PinnedFile,
    component: Option<&manifest::PinnedFile>,
    include_html: bool,
    budget: u64,
    phase: &Cell<impact::UnverifiedReason>,
) -> Result<Generation, ForgeError> {
    phase.set(impact::UnverifiedReason::SnapshotInput);
    captures.restrict_budget(budget)?;
    captures.pinned("snapshot-project", project, manifest::MAX_MANIFEST_BYTES)?;
    let component_path = if let Some(pin) = component {
        phase.set(impact::UnverifiedReason::ComponentInput);
        captures.pinned("snapshot-components", pin, manifest::MAX_MANIFEST_BYTES)?;
        Some(relative_component(project, pin)?)
    } else {
        None
    };
    let generation = prepare_generation(
        &root.join(&project.path),
        component_path.as_deref(),
        true,
        include_html,
        budget.saturating_sub(captures.byte_count()),
        phase,
        Some((&project.expected_sha256, component.map(|pin| pin.expected_sha256.as_str()))),
    )?;
    if generation.prepared.loaded.project_sha256 != project.expected_sha256
        || component.is_some_and(|pin| {
            generation
                .components
                .as_ref()
                .is_none_or(|items| items.manifest_sha256 != pin.expected_sha256)
        })
    {
        return Err(error("snapshot pin changed during capture"));
    }
    captures.reject_cross_aliases(&generation.prepared.captures)?;
    Ok(generation)
}

fn impact_snapshot(generation: &Generation) -> Result<impact::ImpactSnapshot, ForgeError> {
    let rendered = generation
        .rendered
        .as_ref()
        .ok_or_else(|| error("impact requires validated rendered bytes"))?;
    let components = generation
        .components
        .as_ref()
        .map(|items| {
            items
                .instances
                .values()
                .map(|item| {
                    let e = &item.evidence;
                    Ok(impact::ComponentSnapshot {
                        instance_key: e.instance_key.clone(),
                        policy_key: e.policy_key.clone(),
                        topic_key: e.topic_key.clone(),
                        source_sha256: e.source.expected_sha256.clone(),
                        sidecar_sha256: e.component_manifest.expected_sha256.clone(),
                        record_sha256: sha256_hex(&report::encode(e)?),
                        rendered_sha256: item
                            .fragment
                            .as_ref()
                            .map(|fragment| sha256_hex(&fragment.markdown)),
                        answer_keys: e
                            .bindings
                            .iter()
                            .filter_map(|binding| {
                                binding.answer_ref.as_ref().map(|pin| pin.answer_key.clone())
                            })
                            .collect(),
                        question_keys: e
                            .bindings
                            .iter()
                            .filter_map(|binding| binding.question_key.clone())
                            .collect(),
                    })
                })
                .collect::<Result<Vec<_>, ForgeError>>()
        })
        .transpose()?
        .unwrap_or_default();
    Ok(impact::ImpactSnapshot {
        component_manifest_sha256: generation
            .components
            .as_ref()
            .map(|items| items.manifest_sha256.clone()),
        loaded: generation.prepared.loaded.clone(),
        plan: generation.plan.clone(),
        policies: rendered
            .policies
            .iter()
            .map(|policy| (policy.policy_key.clone(), sha256_hex(&policy.markdown)))
            .collect(),
        sections: render::section_hashes(
            &generation.prepared.loaded,
            &generation.plan,
            generation.components.as_ref(),
        )?,
        control_fingerprints: generation.prepared.control_fingerprints()?,
        components,
    })
}

/// Compare explicit snapshots without updating their records or pins.
/// # Errors
/// Invalid/unverifiable comparison returns exit-2 domain error after a safe report.
// The two independent captures and their fail-closed report/publication branch stay together.
#[allow(clippy::too_many_lines)]
pub fn execute_impact(
    path: &Path,
    destination: Option<&Path>,
    format: &AuthorReportFormat,
    include_html: bool,
) -> Result<bool, ForgeError> {
    if include_html && destination.is_none() {
        return Err(error("HTML requires --output-dir"));
    }
    let (root, mut captures, bytes) = capture_request(path)?;
    let request = impact::parse_manifest(&bytes)?;
    // Each snapshot validates independently. Identical pinned project paths are a
    // valid no-op comparison; the request capture is shared without rereading aliases.
    let old_phase = Cell::new(impact::UnverifiedReason::SnapshotInput);
    let new_phase = Cell::new(impact::UnverifiedReason::SnapshotInput);
    let old = pinned_generation(
        &root,
        &mut captures,
        &request.old.project,
        request.old.components.as_ref(),
        false,
        manifest::MAX_TOTAL_BYTES,
        &old_phase,
    );
    // A failed private capture may already have consumed its full allowance.
    // Its partial set is dropped on error; never credit those unknown bytes to
    // the second snapshot. Conservatively reserve the remaining invocation budget.
    let old_size = old.as_ref().map_or_else(
        |_| manifest::MAX_TOTAL_BYTES.saturating_sub(captures.byte_count()),
        |generation| generation.prepared.byte_count(),
    );
    let new = if request.old.project.path == request.new.project.path {
        // Re-capture independently through the authoring loader; request pins are
        // checked against this capture below, avoiding a duplicate CaptureSet role.
        let component = request
            .new
            .components
            .as_ref()
            .map(|pin| {
                new_phase.set(impact::UnverifiedReason::ComponentInput);
                relative_component(&request.new.project, pin)
            })
            .transpose();
        component
            .and_then(|component| {
                prepare_generation(
                    &root.join(&request.new.project.path),
                    component.as_deref(),
                    true,
                    false,
                    manifest::MAX_TOTAL_BYTES
                        .saturating_sub(captures.byte_count())
                        .saturating_sub(old_size),
                    &new_phase,
                    Some((
                        &request.new.project.expected_sha256,
                        request.new.components.as_ref().map(|pin| pin.expected_sha256.as_str()),
                    )),
                )
            })
            .and_then(|generation| {
                if generation.prepared.loaded.project_sha256 != request.new.project.expected_sha256
                    || request.new.components.as_ref().is_some_and(|pin| {
                        generation
                            .components
                            .as_ref()
                            .is_none_or(|items| items.manifest_sha256 != pin.expected_sha256)
                    })
                {
                    return Err(error("new snapshot pins do not match"));
                }
                captures.reject_cross_aliases(&generation.prepared.captures)?;
                Ok(generation)
            })
    } else {
        pinned_generation(
            &root,
            &mut captures,
            &request.new.project,
            request.new.components.as_ref(),
            false,
            manifest::MAX_TOTAL_BYTES.saturating_sub(old_size),
            &new_phase,
        )
    };
    let mut comparison = match (&old, &new) {
        (Ok(old), Ok(new)) => {
            old.prepared.captures.reject_cross_aliases(&new.prepared.captures)?;
            let compared = impact::compare(
                &impact_snapshot(old)?,
                &impact_snapshot(new)?,
                request.correspondence.as_ref(),
            )?;
            old.prepared.verify_inputs()?;
            new.prepared.verify_inputs()?;
            compared
        }
        _ => impact::incomplete_report_with_reason(
            &request.old.project.expected_sha256,
            &request.new.project.expected_sha256,
            if old.is_err() && new.is_err() {
                "both"
            } else if old.is_err() {
                "old"
            } else {
                "new"
            },
            if old.is_err() { old_phase.get() } else { new_phase.get() },
        )?,
    };
    impact::bind_request(&mut comparison, &sha256_hex(&bytes))?;
    let json = impact::render_json_bounded(&comparison, output::MAX_OUTPUT_BYTES)?;
    let text = impact::render_text_bounded(
        &comparison,
        output::MAX_OUTPUT_BYTES.saturating_sub(json.len()),
    )?;
    let mut artifacts = vec![
        artifact("impact.json", json.clone()),
        artifact("impact.txt", text.as_bytes().to_vec()),
    ];
    if include_html {
        artifacts.push(artifact(
            "impact.html",
            html::render_impact_bounded(&comparison, remaining(&artifacts)?)?,
        ));
    }
    check_artifacts(&artifacts)?;
    captures.verify()?;
    if let Some(destination) = destination {
        output::publish(&root, destination, &artifacts)?;
    }
    print_report(&json, &text, *format)?;
    if !comparison.is_complete() {
        return Err(error(
            "authoring comparison is incomplete or unsupported; no unaffected conclusion is available",
        ));
    }
    Ok(comparison.action_required())
}

/// Replay an explicitly pinned generation and publish only new draft records.
/// # Errors
/// Drift, invalid governance, aliases, existing outputs, or publication failures reject handoff.
pub fn execute_handoff(
    path: &Path,
    destination: &Path,
    include_html: bool,
) -> Result<bool, ForgeError> {
    let (root, mut captures, bytes) = capture_request(path)?;
    let request = handoff::parse_manifest(&bytes)?;
    let prior_provenance = captures.pinned(
        "emitted-provenance",
        &request.provenance,
        output::MAX_OUTPUT_BYTES as u64,
    )?;
    let mut drafts = BTreeMap::new();
    for policy in &request.policies {
        let bytes =
            captures.pinned("emitted-draft", &policy.draft, output::MAX_OUTPUT_BYTES as u64)?;
        drafts.insert(policy.author_policy_key.clone(), bytes);
    }
    let mut generation = pinned_generation(
        &root,
        &mut captures,
        &request.project,
        request.components.as_ref(),
        include_html,
        manifest::MAX_TOTAL_BYTES,
        &Cell::new(impact::UnverifiedReason::SnapshotInput),
    )?;
    let rendered = generation.rendered.as_ref().ok_or_else(|| error("handoff requires a build"))?;
    if prior_provenance != rendered.provenance {
        return Err(error("emitted provenance differs from the exact replayed build"));
    }
    for policy in &rendered.policies {
        if drafts.get(&policy.policy_key) != Some(&policy.markdown) {
            return Err(error("emitted draft differs from the exact replayed build"));
        }
    }
    generation.artifacts.extend(handoff::prepare_records_bounded(
        &request,
        rendered,
        remaining(&generation.artifacts)?,
    )?);
    check_artifacts(&generation.artifacts)?;
    captures.verify()?;
    generation.prepared.verify_inputs()?;
    output::publish(&root, destination, &generation.artifacts)?;
    crate::cli::output::write_output(
        "Created draft lifecycle records. Human review remains pending.\n",
        None,
    )
    .map_err(|cause| error(format!("cannot report handoff: {cause}")))?;
    Ok(super::action_required(&generation.plan))
}

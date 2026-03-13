use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ForgeError;
use crate::batch;
use crate::cli::{OutputFormat, Strategy};
use crate::model::{PolicyDocument, PolicySection};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RequirementLocator {
    section_path: String,
    source_line: usize,
    atom_index: usize,
}

fn collect_stable_ids(document: &PolicyDocument) -> HashMap<RequirementLocator, String> {
    let mut map = HashMap::new();
    for section in &document.sections {
        collect_stable_ids_from_section(section, &section.title, &mut map);
    }
    map
}

fn collect_stable_ids_from_section(
    section: &PolicySection,
    section_path: &str,
    map: &mut HashMap<RequirementLocator, String>,
) {
    for requirement in &section.requirements {
        if let Some(stable_id) = &requirement.stable_id {
            map.insert(
                RequirementLocator {
                    section_path: section_path.to_string(),
                    source_line: requirement.source_line,
                    atom_index: requirement.atom_index,
                },
                stable_id.clone(),
            );
        }
    }

    for child in &section.children {
        let child_path = format!("{section_path}/{}", child.title);
        collect_stable_ids_from_section(child, &child_path, map);
    }
}

fn count_substantive_stable_id_changes(
    current: &PolicyDocument,
    baseline: &PolicyDocument,
) -> usize {
    let current_ids = collect_stable_ids(current);
    let baseline_ids = collect_stable_ids(baseline);

    current_ids
        .iter()
        .filter(|(locator, current_id)| {
            baseline_ids.get(*locator).is_some_and(|baseline_id| baseline_id != *current_id)
        })
        .count()
}

/// Convert a max-size value in MB to bytes, with overflow check.
fn max_size_to_bytes(max_size_mb: u64) -> Result<u64, ForgeError> {
    max_size_mb
        .checked_mul(1024 * 1024)
        .ok_or_else(|| ForgeError::Validation("--max-size value is too large".to_string()))
}

/// Validate and resolve `--source-profile` for component strategy.
///
/// Returns `Ok(None)` if no profile was provided (with a warning),
/// `Ok(Some(path))` if valid, or `Err` if empty/whitespace-only or file not found.
fn resolve_source_profile(source_profile: Option<&str>) -> Result<Option<&str>, ForgeError> {
    match source_profile {
        None => Err(ForgeError::InvalidArgument(
            "--source-profile is required for component definitions to produce schema-valid output"
                .to_string(),
        )),
        Some(p) if p.trim().is_empty() => {
            Err(ForgeError::Validation("--source-profile must not be empty".to_string()))
        }
        Some(p) => {
            let profile_path = Path::new(p);
            validate_regular_file(profile_path, "--source-profile")?;
            Ok(Some(p))
        }
    }
}

fn validate_regular_file(path: &Path, flag_name: &str) -> Result<(), ForgeError> {
    if !path.exists() {
        return Err(ForgeError::Validation(format!(
            "{flag_name} path '{}' does not exist (not found)",
            path.display()
        )));
    }
    if !path.is_file() {
        return Err(ForgeError::Validation(format!(
            "{flag_name} path '{}' is not a regular file (is a directory or special file)",
            path.display()
        )));
    }
    Ok(())
}

fn emit_stable_id_change_warning_if_needed(
    input: &Path,
    baseline: &Path,
    max_size_bytes: u64,
) -> Result<(), ForgeError> {
    let baseline_doc = crate::pipeline::prepare_document(baseline, max_size_bytes)?;
    let current_doc = crate::pipeline::prepare_document(input, max_size_bytes)?;
    let changed_count = count_substantive_stable_id_changes(&current_doc, &baseline_doc);

    if changed_count > 0 {
        tracing::warn!(
            changed_count,
            baseline = %baseline.display(),
            current = %input.display(),
            "stable id changed for {changed_count} requirement(s) due to non-whitespace change"
        );
    }

    Ok(())
}

/// Options for the convert subcommand.
pub struct ConvertOptions<'a> {
    pub input: &'a Path,
    pub strategy: &'a Strategy,
    pub format: &'a OutputFormat,
    pub output: Option<&'a Path>,
    pub max_size: u64,
    pub source_profile: Option<&'a str>,
    pub stable_id_baseline: Option<&'a Path>,
    pub import_ssp: Option<&'a str>,
    pub summary: bool,
    pub quiet: bool,
    pub jobs: u16,
}

/// Dispatch convert command: single file → existing path, multiple files → batch mode.
///
/// # Errors
///
/// Returns `ForgeError` if the conversion fails.
pub fn execute_dispatch(input: &[PathBuf], opts: &ConvertOptions<'_>) -> Result<(), ForgeError> {
    // Validate inputs upfront; the empty-input invariant is owned by validate_inputs.
    batch::orchestrator::validate_inputs(input)?;

    if input.len() == 1 {
        // Single-file: delegate to existing execute() unchanged (R6 backward compat)
        let single_opts = ConvertOptions { input: &input[0], ..*opts };
        return execute(&single_opts);
    }

    // Batch mode (2+ files)
    // Warn if --stable-id-baseline was provided (not supported in batch mode)
    if opts.stable_id_baseline.is_some() {
        tracing::warn!("--stable-id-baseline is not supported in batch mode and will be ignored");
    }
    // Warn if --import-ssp was provided (not supported in batch mode)
    if opts.import_ssp.is_some() {
        tracing::warn!("--import-ssp is not supported in batch mode and will be ignored");
    }

    // Validate --output is a dir or absent (FR-014), create if needed (FR-015)
    if let Some(out_path) = opts.output {
        if out_path.exists() && !out_path.is_dir() {
            return Err(ForgeError::BatchConversion(format!(
                "With multiple input files, --output must be a directory, not a file: '{}'",
                out_path.display()
            )));
        }
        if !out_path.exists() {
            std::fs::create_dir_all(out_path).map_err(|e| {
                ForgeError::BatchConversion(format!(
                    "Failed to create output directory '{}': {e}",
                    out_path.display()
                ))
            })?;
        }
    }

    // Validate --source-profile upfront for component strategy (SEC-3, SEC-4, EC-4)
    let resolved_profile = if matches!(opts.strategy, Strategy::Component) {
        resolve_source_profile(opts.source_profile)?
    } else {
        opts.source_profile
    };

    let max_size_bytes = max_size_to_bytes(opts.max_size)?;

    // Derive output paths
    let path_pairs = batch::output_naming::derive_output_paths(input, *opts.format, opts.output);

    // Run batch conversion
    let batch_summary = batch::orchestrator::run_batch_conversion(
        &path_pairs,
        *opts.strategy,
        *opts.format,
        max_size_bytes,
        resolved_profile,
        usize::from(opts.jobs),
    );

    // Print summary to stderr (SEC-7), unless --quiet was specified
    if !opts.quiet {
        let formatted = batch::format_batch_summary(&batch_summary);
        eprint!("{formatted}");
    }

    if batch_summary.has_failures() {
        Err(ForgeError::BatchConversion(format!(
            "{} of {} files failed",
            batch_summary.failed, batch_summary.total_files
        )))
    } else {
        Ok(())
    }
}

/// Execute the convert subcommand (single file).
///
/// # Errors
///
/// Returns `ForgeError` if the conversion fails.
pub fn execute(opts: &ConvertOptions<'_>) -> Result<(), ForgeError> {
    let max_size_bytes = max_size_to_bytes(opts.max_size)?;

    if let Some(baseline) = opts.stable_id_baseline {
        validate_regular_file(baseline, "--stable-id-baseline")?;
        emit_stable_id_change_warning_if_needed(opts.input, baseline, max_size_bytes)?;
    }

    let start = std::time::Instant::now();

    let mut stats = match opts.strategy {
        Strategy::Catalog => crate::pipeline::run_catalog_pipeline(
            opts.input,
            opts.output,
            max_size_bytes,
            opts.format,
            opts.import_ssp,
        )?,
        Strategy::Component => {
            // Runtime validation for --source-profile (SEC-3, SEC-4, EC-4)
            let profile_ref = resolve_source_profile(opts.source_profile)?;
            crate::pipeline::run_component_pipeline(
                opts.input,
                opts.output,
                max_size_bytes,
                profile_ref,
                opts.format,
                opts.import_ssp,
            )?
        }
    };

    if opts.summary && !opts.quiet {
        stats.elapsed = start.elapsed();
        let use_color = std::io::IsTerminal::is_terminal(&std::io::stderr());
        let dashboard = crate::summary::format::format_summary_dashboard(&stats, use_color);
        eprintln!("{dashboard}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::model::{DocumentMetadata, PolicyRequirement};
    use std::path::PathBuf;

    fn single_requirement_doc(section_title: &str, stable_id: &str) -> PolicyDocument {
        PolicyDocument {
            id: "doc".to_string(),
            metadata: DocumentMetadata {
                title: "Title".to_string(),
                version: "1.0.0".to_string(),
                author: None,
                date: None,
                source_path: PathBuf::from("fixture.md"),
                content_hash: None,
            },
            sections: vec![PolicySection {
                title: section_title.to_string(),
                heading_level: 1,
                source_line: 1,
                body_text: None,
                children: vec![],
                requirements: vec![PolicyRequirement {
                    stable_id: Some(stable_id.to_string()),
                    text: "Requirement text".to_string(),
                    source_line: 10,
                    nesting_depth: 0,
                    atom_index: 0,
                    parent_text: None,
                    citations: vec![],
                    modality: None,
                    parameters: vec![],
                }],
            }],
        }
    }

    fn make_opts<'a>(
        input: &'a Path,
        strategy: &'a Strategy,
        format: &'a OutputFormat,
        source_profile: Option<&'a str>,
    ) -> ConvertOptions<'a> {
        ConvertOptions {
            input,
            strategy,
            format,
            output: None,
            max_size: 10,
            source_profile,
            stable_id_baseline: None,
            import_ssp: None,
            summary: false,
            quiet: false,
            jobs: 0,
        }
    }

    #[test]
    fn component_strategy_none_source_profile_errors_with_required_message() {
        let opts = make_opts(Path::new("test.md"), &Strategy::Component, &OutputFormat::Json, None);
        let err = execute(&opts).unwrap_err();
        assert!(
            err.to_string().contains("--source-profile is required"),
            "Missing --source-profile must produce required error. Got: {err}"
        );
    }

    #[test]
    fn component_strategy_empty_source_profile_errors() {
        let opts =
            make_opts(Path::new("test.md"), &Strategy::Component, &OutputFormat::Json, Some(""));
        let err = execute(&opts).unwrap_err();
        assert!(
            err.to_string().contains("--source-profile must not be empty"),
            "Expected empty source-profile error, got: {err}"
        );
    }

    #[test]
    fn component_strategy_whitespace_only_source_profile_errors() {
        let opts =
            make_opts(Path::new("test.md"), &Strategy::Component, &OutputFormat::Json, Some("   "));
        let err = execute(&opts).unwrap_err();
        assert!(
            err.to_string().contains("--source-profile must not be empty"),
            "Expected empty source-profile error for whitespace-only, got: {err}"
        );
    }

    // --- T026: XML format is accepted (no longer rejected) ---

    #[test]
    fn catalog_strategy_xml_format_is_accepted() {
        let opts = make_opts(Path::new("test.md"), &Strategy::Catalog, &OutputFormat::Xml, None);
        let err = execute(&opts).unwrap_err();
        assert!(
            !err.to_string().contains("not yet supported"),
            "XML format should be accepted, not rejected. Got: {err}"
        );
    }

    #[test]
    fn component_strategy_xml_format_is_accepted() {
        // Use a nonexistent source-profile to bypass the --source-profile required check
        // and trigger a file-not-found error (not a "format not yet supported" error).
        let opts = make_opts(
            Path::new("test.md"),
            &Strategy::Component,
            &OutputFormat::Xml,
            Some("nonexistent-profile.json"),
        );
        let err = execute(&opts).unwrap_err();
        assert!(
            !err.to_string().contains("not yet supported"),
            "XML format should be accepted for component strategy. Got: {err}"
        );
    }

    #[test]
    fn yaml_format_is_accepted() {
        let opts = make_opts(Path::new("test.md"), &Strategy::Catalog, &OutputFormat::Yaml, None);
        let err = execute(&opts).unwrap_err();
        assert!(
            !err.to_string().contains("not yet supported"),
            "YAML format should be accepted, not rejected. Got: {err}"
        );
    }

    #[test]
    fn substantive_change_count_detects_matching_locator_id_change() {
        let baseline = single_requirement_doc("Access", "11111111-1111-1111-1111-111111111111");
        let current = single_requirement_doc("Access", "22222222-2222-2222-2222-222222222222");

        let changed_count = count_substantive_stable_id_changes(&current, &baseline);
        assert_eq!(changed_count, 1);
    }

    #[test]
    fn substantive_change_count_ignores_missing_locator_matches() {
        let baseline = single_requirement_doc("Access", "11111111-1111-1111-1111-111111111111");
        let current = single_requirement_doc("Monitoring", "22222222-2222-2222-2222-222222222222");

        let changed_count = count_substantive_stable_id_changes(&current, &baseline);
        assert_eq!(changed_count, 0);
    }
}

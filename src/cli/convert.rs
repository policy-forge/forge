use std::collections::HashMap;
use std::path::Path;

use crate::ForgeError;
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

/// Execute the convert subcommand.
///
/// # Errors
///
/// Returns `ForgeError` if the conversion fails.
pub fn execute(
    input: &Path,
    strategy: &Strategy,
    format: &OutputFormat,
    output: Option<&Path>,
    max_size: u64,
    source_profile: Option<&str>,
    stable_id_baseline: Option<&Path>,
) -> Result<(), ForgeError> {
    let max_size_bytes = max_size
        .checked_mul(1024 * 1024)
        .ok_or_else(|| ForgeError::Validation("--max-size value is too large".to_string()))?;

    if let Some(baseline) = stable_id_baseline {
        validate_regular_file(baseline, "--stable-id-baseline")?;
        emit_stable_id_change_warning_if_needed(input, baseline, max_size_bytes)?;
    }

    match strategy {
        Strategy::Catalog => {
            crate::pipeline::run_catalog_pipeline(input, output, max_size_bytes, format)
        }
        Strategy::Component => {
            // Runtime validation for --source-profile (SEC-3, SEC-4, EC-4)
            let profile_ref = match source_profile {
                None => {
                    tracing::warn!(
                        "--source-profile not provided; control-id mapping will be skipped. The generated Component Definition will have empty control-implementations."
                    );
                    None
                }
                Some(p) if p.trim().is_empty() => {
                    return Err(ForgeError::Validation(
                        "--source-profile must not be empty".to_string(),
                    ));
                }
                Some(p) => {
                    let profile_path = std::path::Path::new(p);
                    validate_regular_file(profile_path, "--source-profile")?;
                    Some(p)
                }
            };
            crate::pipeline::run_component_pipeline(
                input,
                output,
                max_size_bytes,
                profile_ref,
                format,
            )
        }
    }
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

    #[test]
    fn component_strategy_none_source_profile_does_not_error_on_missing_profile() {
        // T013: With source_profile: None, should NOT get source-profile-required error.
        // It will fail on file-not-found (test.md doesn't exist), which proves the profile
        // check is no longer blocking.
        let result = execute(
            Path::new("test.md"),
            &Strategy::Component,
            &OutputFormat::Json,
            None,
            10,
            None,
            None,
        );
        let err = result.unwrap_err();
        // Should NOT be about source-profile — should be about the missing input file
        assert!(
            !err.to_string().contains("--source-profile is required"),
            "Should not require source-profile. Got: {err}"
        );
    }

    #[test]
    fn component_strategy_empty_source_profile_errors() {
        let result = execute(
            Path::new("test.md"),
            &Strategy::Component,
            &OutputFormat::Json,
            None,
            10,
            Some(""),
            None,
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("--source-profile must not be empty"),
            "Expected empty source-profile error, got: {err}"
        );
    }

    #[test]
    fn component_strategy_whitespace_only_source_profile_errors() {
        let result = execute(
            Path::new("test.md"),
            &Strategy::Component,
            &OutputFormat::Json,
            None,
            10,
            Some("   "),
            None,
        );
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("--source-profile must not be empty"),
            "Expected empty source-profile error for whitespace-only, got: {err}"
        );
    }

    // --- T026: XML format is accepted (no longer rejected) ---

    #[test]
    fn catalog_strategy_xml_format_is_accepted() {
        // T026: OutputFormat::Xml should NOT be rejected for catalog strategy.
        // Will fail on file-not-found (test.md doesn't exist), proving the
        // format check no longer blocks XML.
        let result = execute(
            Path::new("test.md"),
            &Strategy::Catalog,
            &OutputFormat::Xml,
            None,
            10,
            None,
            None,
        );
        let err = result.unwrap_err();
        assert!(
            !err.to_string().contains("not yet supported"),
            "XML format should be accepted, not rejected. Got: {err}"
        );
    }

    #[test]
    fn component_strategy_xml_format_is_accepted() {
        // T026: OutputFormat::Xml should NOT be rejected for component strategy.
        let result = execute(
            Path::new("test.md"),
            &Strategy::Component,
            &OutputFormat::Xml,
            None,
            10,
            None,
            None,
        );
        let err = result.unwrap_err();
        assert!(
            !err.to_string().contains("not yet supported"),
            "XML format should be accepted for component strategy. Got: {err}"
        );
    }

    #[test]
    fn yaml_format_is_accepted() {
        // WI-27: YAML format should be accepted (no longer rejected).
        // Will fail on file-not-found (test.md doesn't exist), proving the
        // format check no longer blocks YAML.
        let result = execute(
            Path::new("test.md"),
            &Strategy::Catalog,
            &OutputFormat::Yaml,
            None,
            10,
            None,
            None,
        );
        let err = result.unwrap_err();
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

use std::fs::File;
use std::io::Read as _;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::types::{
    InputFormat, InventoryRequirement, LocationBasis, RequirementInventory, RequirementLocation,
    SourceProvenance,
};
use crate::error::ForgeError;
use crate::model::{PolicyRequirement, PolicySection};

pub(crate) fn build_inventory(
    path: &Path,
    max_size_bytes: u64,
) -> Result<RequirementInventory, ForgeError> {
    let format = input_format(path)?;
    verify_content_format(path, format)?;
    let document = crate::pipeline::prepare_document(path, max_size_bytes)?;
    let location_basis = match format {
        InputFormat::Markdown => LocationBasis::SourceLine,
        InputFormat::Pdf | InputFormat::Docx => LocationBasis::NormalizedExtractedTextLine,
    };
    let label = path.to_string_lossy().into_owned();
    let sha256 = document.metadata.content_hash.ok_or_else(|| {
        ForgeError::MigrationError("source fingerprint was not produced by ingestion".to_string())
    })?;
    validate_source_sha256(&sha256)?;
    let source = SourceProvenance { label: label.clone(), format, sha256, location_basis };

    let mut requirements = Vec::new();
    for section in &document.sections {
        let section_path = escape_section_path_component(&section.title);
        collect_section(section, &section_path, &label, location_basis, &mut requirements)?;
    }
    requirements.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
    validate_unique_ids(&requirements)?;
    Ok(RequirementInventory { source, requirements })
}

fn input_format(path: &Path) -> Result<InputFormat, ForgeError> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "md" | "markdown" => Ok(InputFormat::Markdown),
        "pdf" => Ok(InputFormat::Pdf),
        "docx" => Ok(InputFormat::Docx),
        extension => {
            Err(ForgeError::MigrationError(format!("unsupported policy format '.{extension}'")))
        }
    }
}

fn verify_content_format(path: &Path, declared_format: InputFormat) -> Result<(), ForgeError> {
    let mut file = File::open(path)?;
    let mut header = [0_u8; 5];
    let bytes_read = file.read(&mut header)?;
    let Some(detected_format) = sniff_format(&header[..bytes_read]) else {
        return Ok(());
    };

    if detected_format == declared_format {
        return Ok(());
    }

    Err(ForgeError::MigrationError(format!(
        "source content appears to be {} but its extension declares {}",
        detected_format.as_str(),
        declared_format.as_str()
    )))
}

fn sniff_format(header: &[u8]) -> Option<InputFormat> {
    if header.starts_with(b"%PDF-") {
        Some(InputFormat::Pdf)
    } else if header.starts_with(b"PK\x03\x04") {
        Some(InputFormat::Docx)
    } else {
        None
    }
}

fn validate_source_sha256(sha256: &str) -> Result<(), ForgeError> {
    crate::json_strict::validate_lowercase_sha256("source.sha256", sha256)
        .map_err(ForgeError::MigrationError)
}

/// Escape a title for use as one reversible section-path component.
fn escape_section_path_component(title: &str) -> String {
    let mut escaped = String::with_capacity(title.len());
    append_escaped_section_title(&mut escaped, title);
    escaped
}

/// Append one escaped section title to an existing serialized path.
fn append_escaped_section_title(output: &mut String, title: &str) {
    for character in title.chars() {
        match character {
            '%' => output.push_str("%25"),
            '/' => output.push_str("%2F"),
            _ => output.push(character),
        }
    }
}

fn append_section_path(section_path: &str, child_title: &str) -> String {
    let mut child_path = String::with_capacity(section_path.len() + child_title.len() + 1);
    child_path.push_str(section_path);
    child_path.push('/');
    append_escaped_section_title(&mut child_path, child_title);
    child_path
}

fn collect_section(
    section: &PolicySection,
    section_path: &str,
    file_label: &str,
    location_basis: LocationBasis,
    output: &mut Vec<InventoryRequirement>,
) -> Result<(), ForgeError> {
    for requirement in &section.requirements {
        output.push(inventory_requirement(
            requirement,
            section,
            section_path,
            file_label,
            location_basis,
        )?);
    }
    for child in &section.children {
        let child_path = append_section_path(section_path, &child.title);
        collect_section(child, &child_path, file_label, location_basis, output)?;
    }
    Ok(())
}

fn inventory_requirement(
    requirement: &PolicyRequirement,
    section: &PolicySection,
    section_path: &str,
    file_label: &str,
    location_basis: LocationBasis,
) -> Result<InventoryRequirement, ForgeError> {
    let stable_id = requirement.stable_id.clone().ok_or_else(|| {
        ForgeError::MigrationError(format!(
            "shared pipeline returned a requirement without a stable ID in '{file_label}', section '{}', source line {}",
            section.title, requirement.source_line
        ))
    })?;
    let normalized_text = crate::uuid::normalize_for_hashing(&requirement.text);
    let normalized_text_sha256 = format!("{:x}", Sha256::digest(normalized_text.as_bytes()));
    Ok(InventoryRequirement {
        stable_id,
        normalized_text_sha256,
        normalized_text,
        location: RequirementLocation {
            file_label: file_label.to_string(),
            section_path: section_path.to_string(),
            section_title: section.title.clone(),
            line: requirement.source_line,
            line_basis: location_basis,
            atom_index: requirement.atom_index,
        },
    })
}

fn validate_unique_ids(requirements: &[InventoryRequirement]) -> Result<(), ForgeError> {
    for pair in requirements.windows(2) {
        if pair[0].stable_id == pair[1].stable_id {
            let first = &pair[0].location;
            let second = &pair[1].location;
            return Err(ForgeError::MigrationError(format!(
                "stable-ID integrity anomaly for '{id}': '{first_file}' at {first_path}:{first_line} conflicts with '{second_file}' at {second_path}:{second_line}",
                id = pair[0].stable_id,
                first_file = first.file_label,
                first_path = first.section_path,
                first_line = first.line,
                second_file = second.file_label,
                second_path = second.section_path,
                second_line = second.line
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_title_separator_is_escaped_in_serialized_path() {
        let title_with_separator = append_section_path("Parent", "Access Control / Audit");
        let nested_sections =
            append_section_path(&append_section_path("Parent", "Access Control"), "Audit");

        assert_eq!(title_with_separator, "Parent/Access Control %2F Audit");
        assert_ne!(title_with_separator, nested_sections);
        assert_eq!(title_with_separator.matches('/').count(), 1);
    }

    #[test]
    fn percent_escape_prevents_section_path_encoding_collisions() {
        assert_ne!(
            escape_section_path_component("Access Control / Audit"),
            escape_section_path_component("Access Control %2F Audit"),
        );
    }

    #[test]
    fn duplicate_identifier_error_names_both_locations() {
        let requirements = vec![
            InventoryRequirement {
                stable_id: "AC-1".to_string(),
                normalized_text_sha256: "a".repeat(64),
                normalized_text: "first".to_string(),
                location: RequirementLocation {
                    file_label: "old.md".to_string(),
                    section_path: "Access Control".to_string(),
                    section_title: "Access Control".to_string(),
                    line: 10,
                    line_basis: LocationBasis::SourceLine,
                    atom_index: 0,
                },
            },
            InventoryRequirement {
                stable_id: "AC-1".to_string(),
                normalized_text_sha256: "b".repeat(64),
                normalized_text: "second".to_string(),
                location: RequirementLocation {
                    file_label: "new.md".to_string(),
                    section_path: "Audit".to_string(),
                    section_title: "Audit".to_string(),
                    line: 20,
                    line_basis: LocationBasis::SourceLine,
                    atom_index: 0,
                },
            },
        ];

        let error = validate_unique_ids(&requirements).unwrap_err().to_string();
        for fragment in ["old.md", "Access Control", ":10", "new.md", "Audit", ":20"] {
            assert!(error.contains(fragment), "{error}");
        }
    }

    #[test]
    fn missing_stable_identifier_error_names_requirement_location() {
        let requirement = PolicyRequirement {
            stable_id: None,
            text: "The organization shall protect records.".to_string(),
            source_line: 42,
            nesting_depth: 0,
            atom_index: 0,
            parent_text: None,
            citations: Vec::new(),
            modality: None,
            parameters: Vec::new(),
            parameters_extracted: false,
        };
        let section = PolicySection {
            title: "Records".to_string(),
            heading_level: 1,
            source_line: 1,
            body_text: None,
            children: Vec::new(),
            requirements: Vec::new(),
        };

        let error = inventory_requirement(
            &requirement,
            &section,
            "Records",
            "policy.md",
            LocationBasis::SourceLine,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("policy.md"));
        assert!(error.contains("Records"));
        assert!(error.contains("42"));
    }

    #[test]
    fn sniffed_binary_format_must_match_declared_extension() {
        assert_eq!(sniff_format(b"%PDF-1.7"), Some(InputFormat::Pdf));
        assert_eq!(sniff_format(b"PK\x03\x04"), Some(InputFormat::Docx));
        assert_eq!(sniff_format(b"# Policy"), None);
        assert!(validate_source_sha256(&"a".repeat(64)).is_ok());
        assert!(validate_source_sha256(&"A".repeat(64)).is_err());

        let directory = tempfile::tempdir().unwrap();
        let renamed_pdf = directory.path().join("renamed.md");
        std::fs::write(&renamed_pdf, b"%PDF-1.7").unwrap();
        assert!(
            verify_content_format(&renamed_pdf, InputFormat::Markdown)
                .unwrap_err()
                .to_string()
                .contains("appears to be pdf but its extension declares markdown")
        );
    }

    #[test]
    fn preserves_pipeline_error_variant() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("policy.md");
        std::fs::write(&path, b"# Policy\n- The organization shall protect records.").unwrap();

        assert!(matches!(build_inventory(&path, 1), Err(ForgeError::FileTooLarge { .. })));
    }
}

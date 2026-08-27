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
    let document = crate::pipeline::prepare_document(path, max_size_bytes)
        .map_err(|error| ForgeError::MigrationError(error.to_string()))?;
    let format = input_format(path)?;
    let location_basis = match format {
        InputFormat::Markdown => LocationBasis::SourceLine,
        InputFormat::Pdf | InputFormat::Docx => LocationBasis::NormalizedExtractedTextLine,
    };
    let label = path.to_string_lossy().into_owned();
    let source = SourceProvenance {
        label: label.clone(),
        format,
        sha256: document.metadata.content_hash.ok_or_else(|| {
            ForgeError::MigrationError(
                "source fingerprint was not produced by ingestion".to_string(),
            )
        })?,
        location_basis,
    };

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
        ForgeError::MigrationError(
            "shared pipeline returned a requirement without a stable ID".to_string(),
        )
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
            return Err(ForgeError::MigrationError(format!(
                "stable-ID integrity anomaly for '{}'",
                pair[0].stable_id
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
}

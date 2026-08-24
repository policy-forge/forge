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
        collect_section(section, &section.title, &label, location_basis, &mut requirements)?;
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
        let child_path = format!("{section_path}/{}", child.title);
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

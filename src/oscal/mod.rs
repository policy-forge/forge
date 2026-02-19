//! OSCAL (Open Security Controls Assessment Language) output generation.

pub mod back_matter;
pub mod catalog;
pub mod component_definition;
pub mod implemented_requirements;
pub mod metadata;
pub mod parts;
pub mod profile;
pub mod trace_embedding;

#[cfg(test)]
pub(crate) mod test_utils;

pub use back_matter::{
    BackMatter, BackMatterResource, OscalLink, generate_back_matter, generate_control_links,
};
pub use catalog::{CatalogEnvelope, OscalCatalog, OscalControl, OscalGroup, build_catalog};
pub use component_definition::{
    ComponentDefinition, ComponentDefinitionEnvelope, ComponentDefinitionMetadata,
    DEFAULT_COMPONENT_TITLE, DocumentaryComponent, build_component_definition,
};
pub use metadata::{MetadataOptions, OSCAL_VERSION, OscalMetadata, assemble_metadata};
pub use parts::{OscalPart, OscalProp, build_control_parts, build_control_props, generate_part_id};
pub use profile::{
    ControlSelection, OscalProfile, ProfileImport, ProfileRoot, SelectionMode, build_profile,
    parse_control_ids,
};

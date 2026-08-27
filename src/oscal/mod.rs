//! OSCAL (Open Security Controls Assessment Language) output generation.
//!
//! This module provides types and builders for producing artifacts compliant with
//! [`OSCAL_VERSION`](metadata::OSCAL_VERSION) (currently v1.2.3).
//! including Catalogs, Component Definitions, Profiles, and Assessment Plans.

/// OSCAL Assessment Plan types and builder functions.
pub mod assessment_plan;
/// OSCAL Back Matter generation for reference resources.
pub mod back_matter;
/// OSCAL Catalog builder mapping PolicyDocument to Catalog JSON.
pub mod catalog;
/// OSCAL Component Definition builder for documentary components.
pub mod component_definition;
/// OSCAL implemented-requirements builder for control implementations.
pub mod implemented_requirements;
/// OSCAL metadata assembly with UUID and timestamp generation.
pub mod metadata;
/// OSCAL Part and Prop builders for control content and metadata.
pub mod parts;
/// OSCAL Profile generation with control selection and parameter overrides.
pub mod profile;
/// OSCAL System Security Plan builder with placeholder users.
pub mod ssp;
/// Traceability embedding for source provenance in OSCAL elements.
pub mod trace_embedding;

#[cfg(test)]
pub(crate) mod test_utils;

/// Top-level Assessment Plan JSON envelope and builder.
pub use assessment_plan::{
    AssessmentPlanEnvelope, build_assessment_plan, complete_assessment_plan,
    create_assessment_subjects, derive_ap_output_path, generate_assessment_tasks,
};
/// Back Matter types and generation functions.
pub use back_matter::{
    BackMatter, BackMatterResource, OscalLink, generate_back_matter, generate_control_links,
};
/// Catalog types and builder.
pub use catalog::{CatalogEnvelope, OscalCatalog, OscalControl, OscalGroup, build_catalog};
/// Component Definition types and builder.
pub use component_definition::{
    ComponentDefinition, ComponentDefinitionEnvelope, ComponentDefinitionMetadata,
    DEFAULT_COMPONENT_TITLE, DocumentaryComponent, build_component_definition,
};
/// Implemented-requirement types and builder.
pub use implemented_requirements::{
    ControlImplementation, ImplementedRequirement, build_control_implementations,
};
/// Metadata types and assembly function.
pub use metadata::{MetadataOptions, OSCAL_VERSION, OscalMetadata, assemble_metadata};
/// Part and Prop types with builder functions.
pub use parts::{OscalPart, OscalProp, build_control_parts, generate_part_id};
/// Profile types and builder with control selection.
pub use profile::{
    ControlSelection, IncludeAll, OscalProfile, ProfileImport, ProfileRoot, SelectionMode,
    build_profile, parse_control_ids,
};
/// System Security Plan types and builder.
pub use ssp::{
    AuthorizedUser, ByComponent, DEFAULT_SSP_TITLE, ImplementationStatus, SSP_OSCAL_VERSION,
    SspComponentInput, SspControlImplementation, SspImplementedRequirement, SspMetadata,
    SystemImplementation, SystemSecurityPlanEnvelope, build_ssp, build_ssp_skeleton,
    generate_inventory_items,
};
/// Traceability embedding helpers and constants.
pub use trace_embedding::{
    FORGE_TRACE_NS, LINK_REL_SOURCE, PROP_SOURCE_FILE, PROP_SOURCE_LINE, PROP_SOURCE_SECTION,
    build_trace_link, build_trace_props, embed_trace_in_catalog,
};

//! OSCAL (Open Security Controls Assessment Language) output generation.

pub mod back_matter;
pub mod catalog;
pub mod metadata;

pub use back_matter::{
    BackMatter, BackMatterResource, OscalLink, generate_back_matter, generate_control_links,
};
pub use catalog::{CatalogEnvelope, OscalCatalog, OscalControl, OscalGroup, build_catalog};
pub use metadata::{MetadataOptions, OSCAL_VERSION, OscalMetadata, assemble_metadata};

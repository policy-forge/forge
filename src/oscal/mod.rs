//! OSCAL (Open Security Controls Assessment Language) output generation.

pub mod catalog;
pub mod metadata;

pub use catalog::{CatalogEnvelope, OscalCatalog, OscalControl, OscalGroup, build_catalog};
pub use metadata::{MetadataOptions, OSCAL_VERSION, OscalMetadata, assemble_metadata};

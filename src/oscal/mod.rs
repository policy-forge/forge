//! OSCAL (Open Security Controls Assessment Language) output generation.

pub mod catalog;
pub mod metadata;
pub mod parts;

pub use catalog::{CatalogEnvelope, OscalCatalog, OscalControl, OscalGroup, build_catalog};
pub use metadata::{MetadataOptions, OSCAL_VERSION, OscalMetadata, assemble_metadata};
pub use parts::{OscalPart, OscalProp, build_control_parts, build_control_props, generate_part_id};

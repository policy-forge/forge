//! OSCAL (Open Security Controls Assessment Language) output generation.

pub mod catalog;

pub use catalog::{
    CatalogEnvelope, OscalCatalog, OscalControl, OscalGroup, OscalMetadata, build_catalog,
};

pub mod citation;
pub mod cli;
pub mod error;
pub mod export;
pub mod ingest;
pub mod model;
pub mod oscal;
pub mod parse;
pub mod pipeline;
pub mod uuid;
pub mod validate;

pub use citation::extract_citations;
pub use error::ForgeError;
pub use model::trace::{SourceLocation, TraceError, TraceLink, TraceLinkCollection};
pub use model::{Citation, PolicyDocument, PolicyRequirement, PolicySection};
pub use oscal::{
    BackMatter, BackMatterResource, ComponentDefinition, ComponentDefinitionEnvelope,
    ComponentDefinitionMetadata, DEFAULT_COMPONENT_TITLE, DocumentaryComponent, OscalLink,
    OscalMetadata, OscalPart, OscalProp, assemble_metadata, build_component_definition,
    generate_back_matter, generate_control_links,
};

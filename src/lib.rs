pub mod citation;
pub mod cli;
pub mod error;
pub mod export;
pub mod ingest;
pub mod model;
pub mod oscal;
pub mod parameter;
pub mod parse;
pub mod pipeline;
#[doc(hidden)]
pub mod testing;
pub mod uuid;
pub mod validate;

pub use citation::extract_citations;
pub use error::{ForgeError, exit_code};
pub use model::trace::{SourceLocation, TraceError, TraceLink, TraceLinkCollection};
pub use model::{
    Citation, ConstraintType, Modality, ParameterConstraint, ParameterType, PolicyDocument,
    PolicyParameter, PolicyRequirement, PolicySection,
};
pub use oscal::{
    BackMatter, BackMatterResource, ComponentDefinition, ComponentDefinitionEnvelope,
    ComponentDefinitionMetadata, DEFAULT_COMPONENT_TITLE, DocumentaryComponent, OscalLink,
    OscalMetadata, OscalPart, OscalProp, assemble_metadata, build_component_definition,
    generate_back_matter, generate_control_links,
};
pub use validate::{
    OscalModelType, SchemaError, ValidateError, ValidationResult, check_file_size,
    detect_model_type, load_schema, validate_artifact,
};
